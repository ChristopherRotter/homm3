# Client lane — QUEUE archive

Finished history and informational notices moved out of `QUEUE.md` on 2026-10-24, verbatim, so that the queue holds only what is open. Nothing here is a task. Grep it when a doc, a decision or a commit cites an entry (`git grep -n 'T21b' docs/lanes/client/QUEUE-ARCHIVE.md`). New finished items are appended at the top of their section.

## Notices from other lanes and finished items, newest first

- 2026-11-12 (E27 M1 T1a2 done: its task entry verbatim, then the town lane's inbox note it
  acted on, verbatim: `multiplayer_dialog_init` 0x50E050 read and corrected, design §2.6; the
  record in `E27-log.md`, "M1 T1a2"):

  #### E27 M1 T1a2 — the catalogue, part 1b: Multiplayer, the hot seat's names, High Scores, Credits

  - **Scope:** T1a's second half (cut with the user, 2026-11-12, decision 9; T1a1's reading is
    `E27-design.md` §2, its entry points named there): Multiplayer (`multiplayer_dialog_run`
    0x4F0590: the dialog 0x50E050, its modes, the hot seat's names dialog; the network modes decided
    as a `CLT-Q` with a recommendation, CLT-Q83; `ProgressBar`'s use, §2.1, with them), High Scores
    (`high_scores_show` 0x4E9110: its tabs, `HSANIM` and `HSLOOP`, the high-score manager 0x6993CC),
    Credits (`credits_screen` 0x4EDDA0: its text table `Credits.txt`, whether a clip plays, the
    copies of `mainmenu.pcx` around it). Per screen as T1a1: the constructor, its items in order,
    the assets by name, the texts by table and record (never the text), the sounds, the entry points
    and the transitions; confidence stated; every oddity a question. Into `E27-design.md` §2 (new
    subsections after §2.5).
  - **Acceptance:** as T1a1's (every screen listed with its addresses, items, assets, texts, sounds
    and transitions; the questions filed, asked when the user is present; the map's entries and
    `check-exe-map`); the Definition of Done (docs).
  - **Depends on:** T1a1 (done). **Status:** ready. The original data: `Heroes3.exe` and the LODs
    (the text tables in `original-data/extracted/`); `original-data/shell-videos.vid` holds
    `HSANIM` and `HSLOOP` for their headers.

  - 2026-11-12 (the town lane, E09 T25a, #873; **for T1a2**): this queue's T1a2 entry cites
    0x50E050, which `docs/exe/map.toml` lacked: #876's client-only CI skips `check-exe-map`, so
    the next full CI (#873's) failed on `master`'s text. #873 adds `multiplayer_dialog_init`
    (0x50E050, size 0xCFC, `inferred`: built on `multiplayer_dialog_run`'s stack, its only caller,
    run by `dialog_run_modal`, then destroyed; its calls not read). T1a2 corrects it when it
    reads the dialog; run `cargo xtask check-exe-map` before a docs-only push that cites an address.

- 2026-11-12 (E27 M0, the refinement with the user: the inbox's E27 entry placed in the epic's
  scope, M2 and M3; the entry verbatim):

  - **E27** (in its draft since M7 T4): the client's options (the volume settings 0x5996C0, a track
    resuming 0x59AA82, CLT-Q25's step back), video (the intro videos, the video archives), CLT-Q2's
    archive set by the LODs' tables (NOTES, "Assets and archives", "Options and layouts").

- 2026-11-12 (E17 M7 T4 done, **E17 closed**; the queue's tasks section and T4's entry verbatim,
  then the inbox entries placed at E17's closure, verbatim: the town lane's harness note acted on
  (`xdd.dll` rebuilt; informational), the encoding-3 drawer note (informational, the user), the
  harness check's growth (to the adventure lane's inbox), CLT-Q2 (to E27), CLT-Q4 (with E24 M4
  T5), "Later" (E27, E28, E29) and "The live client" (E28); their new lines in `QUEUE.md`; the
  record in `E17-log.md`, "M7 T4"):

### E17 M7 — the queue's tasks section at E17's end

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27, M5 2026-10-31, M6 2026-11-10; ADR-0020, ADR-0024; the battle
client since the split of 2026-10-23, the adventure map client being the map-client lane's
E24). M7 (the battle client's last steps) refined at M6's closure (T7c, 2026-11-10, with the
user): M6's answers built, the panel buttons' own loop, the battle as an entry a host's window
runs, then the closure, **which closes E17** (the user: ranked play, the pause and the server
browser wait for E27 and a refinement after it). Its tasks below, in order, each cut with the
user at its start when it outgrows half a context. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-battle-client -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M7 T4 — M7's and E17's closure

### The inbox entries placed at E17's closure (M7 T4)

- **Scope:** a fresh reviewer; the captures where M7 changed the battle screen; the review list
  with the user; E17's open threads placed (the epic's "Later": HD layouts, graphics options,
  the client's options, video, M6 T5e; this inbox's waiting entries; ranked play's notes for the
  refinement after E27); E17 closed (`docs/BACKLOG.md`'s row, and E27's draft, which still names
  E17 M7 for the lobby and the browser, by a `shared/` pull request).
- **Acceptance:** the review list accepted; every open thread placed; E17 done.
- **Depends on:** T1 to T3 (done 2026-11-11). **Status:** next.

- 2026-11-11 (the town lane, E09 T23, branch `town/e09-t23`, `cross-lane`; **the capture harness
  changed**; `clients/capture/README.md`, "Forced rolls" and "Growth, the garrison hero and a
  town's hire"): xdd.dll's table gains twelve adventure-map sites (four forced: `week_chance`,
  `week_creature`, `month_kind`, `month_creature`; eight logged: the neutral garrisons' and the
  Portal's draws), so `h3drive.ROLLS` and `LOGGED` hold 30 calls: a battle sequence checks that
  xdd.dll redirected all 30, and a stale `xdd.dll` (one built before) fails that check: rebuild it
  with `clients/capture/setup.sh`. `h3adv` gains a recipe's and a step's `rolls`, `roll` and
  `date`, the reads `draws`, `turn_draws`, `kinds`, the town parts `counts`, `count:<i>`,
  `dwellings`, `portal`, `garrison_hero`, `hire:creatures`, the town screen actions `hero_in` and
  `building`; `h3mem`'s town record gains `recruits`, `portal`, `dwellings`. Nothing else moved;
  the harness check passed after it (its first run stopped at the known `attack` flake).

- 2026-10-27 (the map client lane, E24 M2 T3b; for you when a translucent DEF of encoding 3
  appears): `draw_frame_blend` 0x47D4F0 is read (`docs/exe/map.toml`; E24 design §3.8): it blends
  encoding 3's literal pixels half and half with the screen as 0x47EF60 blends encoding 1's, and
  skips its runs (both callers pass no colour). `h3_render::blit`'s `Drawer::Translucent` still
  draws encoding 3 plain (*provisional*: no magic animation has it); the map view blends with its
  own drawer (`h3_map_view::draw::blend`), so nothing in the foundation changed.

- 2026-10-19 (E17 H1, H2; the adventure lane, E08 T22a1): the harness's check
  (`capture/checks/harness-check.toml`) can gain a monster's offers, a pile's question and a walk
  below: the tools are in. NOTES, "The adventure lane's harness".

- CLT-Q2 (2026-10-12, 2026-10-19): the table-only LOD reader is in `h3-formats`: switch the
  archive set to the directories and a batch's ranges, `h3-view gate` measuring it. NOTES,
  "Assets and archives".

- CLT-Q4: the background variants 0 and 1 (read from the wire since E17 M7 T3b1; not recorded:
  a situation needs trees, mountains or dead vegetation around the battle). NOTES,
  "Battlefields and boats".

- Later (placed at E17 M7 T4): graphics options (the user, at M1), an HD layout's origin and viewport.
  NOTES, "Options and layouts". (The adventure map's object draw order, research item 9, went to
  the map-client lane at the split, 2026-10-23.)

- The live client (the server lane, 2026-10-09 to 13): M5 T6 and T7 took the record file, the
  content check, a server restart and accounts; the ladders, the pause by agreement and the
  server browser are not E17's (the user, 2026-11-10: after E27, by a refinement of their own;
  the first ranked play through the website, E21). NOTES, "The live client".

- 2026-11-11 (E17 M7 T3c done, T3 done; the entry verbatim; the record in `E17-log.md`,
  "M7 T3c"):

### E17 M7 T3 — the battle as a hosted entry

- **Scope:** the map client lane's request for E24 M4 T5 (the inbox of 2026-10-31, now in
  `QUEUE-ARCHIVE.md`): the battle as an element a host's window runs (the shape agreed with the
  user for both lanes, 2026-11-11: the host owns the window, its loop and the frame, the battle
  drawn natively as any element of it; design §13.2), started from a whole game's battle
  messages and handed back with its outcome. **Cut in four** with the user (2026-11-11):
  - **T3a1 done**, **T3a2 done** (2026-11-11, branch `claude/ecstatic-thompson-cjowg2`; T3a2
    checked with the data staged by the next session): the crate `h3-battle-client` (h3-view's
    binary until then), the element `window::Hosted` with `h3-view window` its host.
  - **T3b**, cut in two with the user (2026-11-11):
    - **T3b1 done** (2026-11-11, branch `client/e17-m7-t3b1`; design §13.3): a whole game's
      record's battle played and handed back with its outcome; the wire's picture variant and
      colours read.
    - **T3b2 done** (2026-11-11, branch `client/e17-m7-t3b2` after #851; design §13.4,
      §13.5): the same battle played live through the host's game (`game_seat::GameSeat`,
      `Ground::load_game`; a test host's real game to its result and outcome); the hero's battle
      picture by the map's sex in the client's profile (CLT-Q78 (b)).
  - **T3c done** (2026-11-11, kept whole with the user, branch `client/e17-m7-t3c`; design
    §13.6): the sound on the host's (`audio::HostSound`), the music through the common
    `h3_ui::music` (the end's tune after the combat music's 900 ms fade-out), the focus lost and
    regained, the frame widened by `h3_render::screen::Widening` (0.33 ms a frame against 3.5).
- **Acceptance:** `h3-view window` runs its battles through the entry with `drawn` all the
  same; a test host plays a whole game's battle to its result and gets the outcome back; the map
  client lane's inbox told the entry's API.
- **Depends on:** E16 M5 part 1's whole-game record (merged). **Status:** done 2026-11-11 (T3a,
  T3b and T3c).

- 2026-11-10 (E17 M6 T7c, M6's closure: the inbox's informational entries, the done ones and
  those M7 T3 takes, verbatim; the adventure lane's `random_between` note done in
  `E17-design.md` §10.2 and the runbook's `catapult_tie` row; the record in `E17-log.md`,
  "M6 T7c"):

  - 2026-11-07 (the town lane, E09 T14b, branch `town/e09-t14`; the capture harness, informational):
    `h3adv` plays a walk's `actions` on the town screen when it opens (then closes what they left
    open), with the new actions `hall = true` (a click on the shown hall building, items 10 to 13: the
    hall, vtable 0x6437A0) and `slot = <building id>` (the hall's slot showing it: the building's
    dialog, 0x643944; `click = ["build"]` its Build 30722, `cancel` 30721), and the reads
    `town:x,y:mark`, `hall:slot:<id>`, `build:button`, `build:text`, `screen`, `town_screen_closed`;
    `h3mem`'s town record gains `mark` (+0x2), `h3lod` gains `building_names`. A walk without
    `actions` behaves as before. Learned: the panel's hall picture opens nothing; a map whose players
    are all allies ends at a visit's close (an enemy is needed). The harness check passed after.
    `clients/capture/README.md`, "The hall and the build".

  - 2026-11-07 (the adventure lane, E08 T100b, branch `adventure/e08-t100b`; the capture harness, informational): `h3adv` gains a `key` step's `repeat = { most, first, until = { with } }` (the key pressed again and again, a row each: `presses`, `presses:<columns>`, `presses:changed`), a recipe's `[places]` (`hero:place`, `hero:at`), a step `look = "<hero>"` (the view centred by the minimap, nobody selected; its `before:` reads before the scroll), `walk = "off"` (the first open neighbour), `only_if`'s `empty` and `occupied`, and the observations `whirlpools` (`h3mem.whirlpools`: game +0x4E77C's vector, begin and end at +0x4E780), `tile:x0,y0..x1,y1`, `tile:from`, `tile:here`, `tile:hero[<name>]`, `answers`, `selected`, `target`. No behaviour of an existing step changes. Observed: the selected hero is lifted off its tile's record after its move and at its turn's start, and a scroll of the view puts it back (a tile read depends on the last scroll). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".

  - 2026-11-06 (the adventure lane, E08 T100a, branch `adventure/e08-t100a`; the capture harness, informational): `h3adv` gains a step's `replay` (Replay Opponent Turn: the panel's Adventure Options button, the map dialog's item 9; its dialog vtable 0x63A610, item 5; played on the frozen clock, stepped 20 ms, the flag 0x696A04; `dumps = "every"`) and `at_opening` (reads as a step's first battle opens), `only_if = { at = [x, y] }`, `again_if = { marks = "any" }`, and the observations `replay`, `replay:views`, `replay:watched`, `replay:options`, `view:before`, `view:after`, `view:size`, `explored:x0,y0..x1,y1`, `at_opening:`, `battles`, `exchange`, `army:rows`, `hero:in_boat`. Behaviour changes: a `key` step plays what the key brings by the step's `answer`, `battle` and `level_up` (it stopped at a battle before); a skipped step leaves a field another step filled (it wrote "not played" over it). `clients/capture/README.md`, "Teleporters and Replay Opponent Turn".

  - 2026-11-06 (the adventure lane, E08 T98a2, branch `adventure/e08-t98a2`; informational, a doc fix): **`random_between` 0x50B230 draws nothing when its bounds are equal** (read whole: `cmp esi, edi; jne` returns the bound at 0x50B23A before `crt_rand`; a high bound below the low also returns without a draw). So the Catapult's tie of one weakest wall (`catapult_tie_draw` 0x473DA0, random(1, k) with k = 1) takes no draw: `docs/exe/map.toml`'s entry is corrected; `E17-design.md` §10.2 still says "drawn even for k = 1" (line 4420), and the harness's forced `catapult_tie` (`clients/capture/README.md`) may assume a draw there. Survey `docs/rules/surveys/E08-teleporters.md` §4.1.

  - 2026-11-03 (the adventure lane, E08 T90, branch `adventure/e08-t90`; the capture harness, informational): `h3adv` and `h3mem` gain the dwellings' records (`dwelling:x,y`, game +0x4E39C), `teams`, `objects:` and `access:` (a rectangle's map objects with class and subclass, its tiles' blocked and visitable bits), `camp:x,y`, the hire dialog as a screen (vtable 0x640C4C; `hire`, `hire:opening`, `hire:after`; actions `click = ["max", "recruit"]`, `amount = <n>`), a step's `hint = [x, y]` and `popup = [x, y]`, `opening:defender`, `ai_turns`. Behaviour change: `end_turn` now waits out a computer player's turn (it returned during it); nothing changes on maps with humans only. `clients/capture/README.md`, "Dwellings, the hire dialog and a computer player's turn".

  - 2026-11-03 (the town lane, E09 T3b, branch `town/e09-t3b`; the capture harness, informational
    unless you play battles with `attack`): `h3adv` gains the town reads (`town:x,y:owner`,
    `visitor`, `built`, `every`, `garrison`, `slot:N`; `towns:every_vs_built`,
    `towns:owned_garrisons`), `hero[NAME]:slot:N`, `opening:slot:N:<side>`, `town_screen`,
    `unknown_screens`, a `new_game = true` step, the town screen (vtable 0x64372C, closed by item
    30720) and the level-up dialog (0x63FE60, `level_up`) as known screens, and a list of creatures
    in the table battle plan's `attack`. Behaviour changes: `play_turns` reads the turn again after
    its 0.5 s wait (an order went into the next stack's turn), a retreat waits out the retreating
    side's Catapult, a `text:` literal ending in `: ` joins the next value without `; `, and `batch
    --map` fills the template beside the map when its stamp matches (no cargo build). A town's
    visitor (+0x10) reads −1 while its hero is the selected one. The harness check passed before and
    after. `clients/capture/README.md`, "Towns (E09 T3b)".

  - 2026-11-02 (the battle lane, E26 T5; for a computer side in your battles, when you add one):
    call the fair system actor `h3_battle_ai::choose_fair` by default (ADR-0027 point 4 as
    amended; the user, AI-Q10), the all-seeing `choose` only as a game's option.

  - 2026-11-02 (the battle lane, its inbox; **two answers for you**): (1) `summon-turn`'s round 2
    (your E17 M5 T5b2 note) is no engine difference: `seq-summon.toml` loads `base` alone, where the
    elementals are neutral (speed 6) and the Swordsmen native on grass (5 + 1), so the tie goes to
    the lower slot; with `ab` loaded the elementals are Conflux natives (7) and the engine's round 2
    starts with them, as the original's (SoD). Load `["base", "ab", "sod"]` with `sod:faithful` (as
    `seq-fear.toml`) where a scenario has to match the SoD capture. (2) `h3_formats::text::TextTable`
    now makes each run of quotes in a *quoted* field one, as the executable's readers
    (`installation.md` §4); `h3_ui::text::pack`'s `one_quote` stays harmless, and is redundant for
    quoted fields; `E17-design.md` §7.19's "`h3_formats::text` keeps the doubled quotes" is history.

  - 2026-10-31 (the map client lane, E24 M4 T0; **for E24 M4 T5, late in M4**): the user (M4 T0):
    a battle inside a whole game is drawn by the battle client's viewer in the map viewer's window,
    as the original switches screens, and hands back to the map with its result. The map viewer
    (`h3-map-view`'s binary, or a binary over both libraries) hosts it, and needs from `h3-view` a
    library entry: a battle started from a game record's battle messages (the protocol's battle
    types as the server lane carries them in a whole game, E16 M5; its background and colours from
    the game, CLT-Q4, CLT-Q6), run in a window, a clock and an audio output the host owns, played
    to its result window, then given back. How and when it fits M6 and M7 is yours; this lane
    builds its T5 once the entry is there.

  - 2026-10-31 (the map client lane, E24 M3 T7b2, the review's fixes; for information, nothing to
    do): four changes in the common crates. `h3_ui::paint::draw` saturates a text item's clip and
    a label's colour, and `h3_ui::ui::Dialog::shadow_draws` its strips (a crafted layout at the
    integers' end panicked: the release build checks overflow); `h3_render::blit::darken` cuts its
    rectangle to the screen before its loop (the same pixels); `h3_ui::text::Catalogue::known` is
    new (additive: none for a key no pack has; `text` calls it). Every drawn frame is the same.
    A test now checks that every pixel a drawer changes lies inside `paint::reach`.

  - 2026-10-30 (the map client lane, E24 M3 T6b3; for the battle window's sound when next touched):
    the common crates gained the original's sound without the focus (`docs/epics/E24-design.md`
    §4.15: `sound_focus_lost` 0x599C40 and `sound_focus_regained` 0x599B90, the same sound manager
    in a battle): `h3_ui::music::Music::focus` and `Queue::focus` (the music faded out by the stub
    thread once its lock is free, a track opened meanwhile held silent, asked for again at the
    return) and `h3_sdl::Audio::hold` (a stream held where it is: the effects playing at the loss
    resume at the return). `h3-map-view`'s window follows `Window::focused()` with them; `h3-view`
    plays on without the focus. Nothing changed for an existing caller. The harness: `space` is a key
    a sequence may press.

  - 2026-10-28 (the map client lane, E24 M3 T2b1; additive, nothing of yours changed): **the
    widening by a table.** `h3_render::screen::Widening` holds every pixel of a format widened
    once (`Screen::xrgb`'s values, 65 536 entries) and widens a screen into a kept vector
    (`widen`) or onto part of a frame (`widen_onto`); also new: `Screen::cut`, `Screen::copy_rect`
    and `h3_ui::cursor::Cursor::rect`. In the map's window the per-pixel `xrgb` was 62 % of a
    frame's cost (`docs/epics/E24-design.md` §4.2); `h3-view`'s window widens every frame the same
    way and could take it.

  - 2026-10-27 (the map client lane, E24 M2 T6a2; for the battle's end tune when its sound is next
    touched): **the music thread read whole** (`docs/exe/map.toml`: `play_music` 0x59ACB0,
    `music_thread` 0x59A840, `music_stop_fade` 0x59B080; E24 design §3.11, "The music's thread").
    For `h3-view`: the end's tune does not cut the combat music; the thread fades it out first
    (900 ms by tenths from its volume, its place kept), then opens the tune, so the tune starts
    900 ms after `play_music`, and a battle ended within 900 ms of the music's start waits for the
    fade-in's end too (the thread holds the manager's +0xA8 through it). `h3_map_view::music`
    models the thread (looping tracks only: a track played once starts at its volume from its
    start and is never resumed), a candidate for M3 T0's common crate.

  - 2026-10-24 (the battle lane, CLT-Q4/Q6, `shared/clt-q4-q6-seam` and `battle/clt-q4-q6-carry`,
    both `cross-lane` with the user's consent; **for you**): **the wire carries the viewers' facts**
    (protocol v35): `Observation.background` (0 to 2, the battlefield picture's variant on dirt,
    grass and snow; `None` not known: take the third picture, as now), `SideView.colour` (the
    side's player colour, 0 red to 7 pink; `None` no player or not known: red for the attacker,
    blue for the defender, as now) and `Hero.sex` (`wire::Sex`; `None` the hero type's own, the
    asset tables' `[[hero]]`). Your wire literals got `background: None`, `colour: None` and
    `sex: None` (`h3-view`'s `record.rs`, the mouse, scene and panel tests); nothing reads them yet.
    The scenarios and the battle replay file (format v31: `background`, `attacker_colour`,
    `defender_colour`, a hero's `sex`, each omitted when not known) may name them; the adventure
    map's battles will set them (the adventure lane's inbox). CLT-Q4 and CLT-Q6 are answered.

- 2026-11-10 (the two server-lane inbox entries of 2026-11-09 done after E17 M6 T7a: the comment
  renamed, the protocol v38 readers and CLT-STUB-3; the entries verbatim; the record in
  `E17-log.md`, "After M6 T7a: the protocol v38 readers"):

  - 2026-11-09 (the server lane; informational): `h3_cli::load_ruleset`, which `h3-view`'s
    `scenario.rs` copies (its comment names it), is `h3_files::load_ruleset` now (the new crate
    `h3-files`, the user's decision). Depending on it would pull `h3-sim` and the adventure crates
    into the client; the copy may stay, its comment's name is the one to update.

  - 2026-11-09 (the server lane, the inbox's CLT-STUB-3 and CLT-Q20; protocol v38, record format
    v3, branch `claude/server-lane-continuation-seqai3`): **both done in `h3-protocol`'s session; your
    readers to adapt.** (1) Every viewer gets its last `Snapshot` (the battle as it ended, as it sees
    it) right before `GameEnded`: a seat reads its own hero's Eagle Eye picks from that snapshot's
    `OwnHero::learned` (`results::served`, CLT-STUB-3). (2) When the game starts (every seat held),
    every viewer watching then gets one `Events` message with the battle's start's events (a tactics
    phase, the first turn, a freeze, a town's turrets and Catapult acting in round 1), before any
    `YourTurn` or command's events; a seat re-taken or a spectator joining later never gets it. The
    join's `Snapshot` is taken after the start, so it already shows these events' effects (the
    engine has no state before its start). So a record's first `Events` is no longer a command's:
    `anim::commands` and the readers that count one `Events` per command (`log.rs`,
    `sequences.rs`, `motion.rs`) see one more message at the start and one more `Snapshot` at the
    end. The record header's `format` is 3; `RECORD_FORMAT_VERSION` refuses format 2.

- 2026-11-07 (E17 M6 T6c2 done, T6 done; the entry verbatim; the record in `E17-log.md`,
  "M6 T6c2"):

### E17 M6 — the queue's header at M6's end

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27, M5 2026-10-31; ADR-0020, ADR-0024; the battle client since the split
of 2026-10-23, the adventure map client being the map-client lane's E24). M6 (the battle screen
complete) refined at M5's closure (T8c, 2026-10-31, with the user): what a played battle shows
of the original's battle screen that M5 did not build, M5's gaps and the inbox's waiting items;
its tasks below, in order, each cut with the user at its start when it outgrows half a context.
M7 (ranked play and the server browser) follows. The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M6 T7 — M6's closure (done 2026-11-10, T7c)

- **Scope:** a fresh reviewer, every capture afresh, the review list, M7 refined with the user
  (ranked play and the server browser: the ladders, the pause by agreement, the browser;
  `NOTES.md`, "The live client"), HD and graphics options placed.
- **Acceptance:** the review list accepted; every capture as recorded or its difference listed;
  M7's tasks in this queue; M6 done. **Depends on:** T1 to T6 (done).
- **Cut** in three by the user (2026-11-09): **T7a done** (2026-11-09/10: R4-9 and every capture
  afresh; the epic's log); the protocol v38 readers done (2026-11-10, #818). **T7b done**
  (2026-11-10: 12 reviewers, 6 fixers; the epic's log). **T7c** the review list with the user
  (M6's items 1 to 62), the new questions (CLT-Q70 to Q77) and T7b's left items (the log), M7
  refined (ranked play and the server browser, HD and the graphics options placed), M6 closed
  (its scope to `E17-archive.md`). **Status:** done (T7c, 2026-11-10).

### E17 M6 T6 — the inbox's waiting items

- **Scope:** the items that waited for a free slot (their entries in `QUEUE-ARCHIVE.md`,
  2026-10-31): the deck and the two boats as capture-set situations (CLT-Q5: a hero beside an
  empty boat boards it in the game; `NOTES.md`, "Battlefields and boats"); `h3-bink` decoding as
  `BINKW32.DLL` (CLT-Q30: its float transform in integer arithmetic, checked against the DLL's
  planes; the captures' listed DCT pixels then go); the Demons' dissolve re-investigated (the
  user, 2026-10-29; design §7.25); the compositor's dialog items through `h3_ui::paint`
  (`h3-view`'s copy removed); `tests/purity.rs`'s three holes (R2-15: the map viewer's fixes);
  R4-2 (`drawn`'s digest of what the masks depend on) and R4-9 (a stricter shadow chain in
  `h3cap.redraws`).
- **Acceptance:** each done or its reason recorded; `drawn` and the captures it touches as
  recorded; `check.sh`. **Depends on:** nothing (independent of T1 to T5). **Status:** cut in
  three by the user (2026-11-07): **T6a** done (the purity test's holes, the dialog items
  through `h3_ui::paint`, R4-2; no capture in the original); **T6b** the recordings, cut in two
  by the user (2026-11-07): **T6b1** done (the deck and the two boats as situations, CLT-Q5),
  **T6b2** done (the Demons' dissolve, CLT-Q67; R4-9 moved into T7); **T6c** `h3-bink` as
  `BINKW32.DLL` (CLT-Q30), cut in two by the user (2026-11-07): **T6c1** done (the decoder:
  the ten clips and the samples exact against the DLL's planes; CLT-Q68 answered fix; design
  §7.27); **T6c2** done (2026-11-07): the captures' listed DCT pixels (`result-retreat` at 2 700 and
  3 000, `defended`, `auto-retreat-flee` and `-surrender`; `end`'s `Win3` was exact) captured
  again, every clip frame exact, `drawn` recorded, `TRAPS.md`'s CLT-Q30 line, CLT-Q30 closed.
  T6 done.

- 2026-11-07 (E17 M6 T5 done: T5e deferred by the user at its start to when the battle
  client runs inside an adventure game, the epic's "Later"; the entry verbatim):

### E17 M6 T5 — M5's gaps

- **Scope:** what M5's review list (`E17-log.md`, "M5 review list") names as not built, each as
  far as a reading allows: the book's refusal messages (items 5 and 9: `hero_book_button`'s
  refusals and the aim's entry's other messages), the Auto Combat bot's casts and its retreat
  question by T4's options (item 24), Force Field's and Fire Wall's aim (item 84), Eagle Eye's
  message and the experience given after the window (item 79), the Ogre Magi's Bloodlust glow
  (item 42), the effects not drawn yet and their sounds' waits (item 45: the Wraiths' mana
  drain, regeneration, Poison's damage, Death Blow, Ice Bolt's explosion, the Familiars'
  channel).
- **Acceptance:** each built and recorded, or its reason in M6's review list; `check.sh`.
- **Depends on:** T4 (the Auto Combat options). **Status:** cut in four by the user
  (2026-11-05): **T5a** done (the book's refusals, the aim's other messages, the walls' aim;
  design §12.17); **T5b** cut in two by the user (2026-11-05): **T5b1** done (the window's
  computer the engine's original one, `--sight`; its casts for the human's hero; design §12.18;
  CLT-Q61, Q62 answered); **T5b2** cut in two by the user (2026-11-05): **T5b2a** done (the
  retreat question and the offer; design §12.19; the engine's part #745); **T5b2b** done (kept
  whole by the user: CLT-Q61's stop at the cast's end, a question at a later turn, a human's turn
  right after an answer; design §12.20; CLT-Q64 asked); **T5c** done (kept whole by the user:
  Eagle Eye's message and the box with pictures, the experience after the window nothing to
  show; design §12.21; CLT-Q65, Q66 asked); **T5d** the effects not drawn yet, their sounds'
  waits and the Ogre Magi's Bloodlust glow (items 42, 45), cut in two by the user (2026-11-06):
  T5d1 done (design §12.22); **T5d2** the strikes' and casts' effects, cut in two by the user
  (2026-11-06): T5d2a done (Death Blow and the Ogre Magi's Bloodlust glow, item 42; design
  §12.23); T5d2b done (the hero's cast's: Ice Bolt's explosion and the Familiars' channel;
  design §12.24);
  **T5e** (added by the user at T5c, 2026-11-06) the captured artifacts' message after Eagle
  Eye's (`captured_artifacts_message` 0x4772B0, design §12.21: GENRLTXT 31, eight artifact
  pictures a box, the pickup sound; its reading finished then): the heroes' worn artifacts and
  the backpack the client's heroes do not carry yet (the scenario's and the wire's), the
  capture's order HER-ART-004 (`h3_heroes::next_capture`), recorded.

- 2026-11-05 (E17 M6 T4c done, T4 done; the entry verbatim; the record in `E17-log.md`, "M6 T4c"):

### E17 M6 T4 — the combat options dialog

- **Scope:** the panel's Options (O): the dialog read (its items, keys and defaults), drawn and
  in effect: each setting the original has (the hex grid; the movement and cursor shadows, the
  hex shade design §9.11 does not model yet; the combat speed, whose factors design §7.1 holds;
  the Auto Combat options restricting the window's bot; the others as read), kept in the
  client's settings file (beside `keys.toml`, *provisional*); recorded with settings on.
- **Acceptance:** the dialog and each setting's drawing exact against recordings or listed;
  `check.sh`. **Depends on:** T1 (done). **Status:** cut in three by the user (2026-11-04);
  T4a done (design §12.12); T4b done (cut in three by the user, 2026-11-04: T4b1 §12.13, T4b2a
  §12.14, T4b2b cut in two: T4b2b1 and T4b2b2 §12.15, the army info windows):
  - **T4c:** the hex grid, the movement and cursor shadows drawn (`draw_hex_overlays` 0x493930's
    options, 0x493EA0), recorded with the settings on; the keys F6 to F8 (`battle_set_shades`
    0x479FC0, saved at once). The spell book's page turns ("Animate SpellBook", kept by T4a)
    stay unbuilt (their videos), listed at M6's closure.

- 2026-11-03 (E17 M6 T3c2 done; the entry verbatim; its capture is T3c3 in the queue):

### E17 M6 T3c2 — the descriptions' overrides, the Alliance and a siege town's buildings

- **Scope:** T3c's rest (T3c1 done 2026-11-03: the hero's lists and the unclamped value, design
  §12.9; T3c's entry in `QUEUE-ARCHIVE.md`), from `Observation.morale_and_luck` and
  `Siege.bonuses` (design §12.4): the Hourglass of the Evil Hour's line (82, alone; a
  `LuckAtMost` reaching the side, either hero's), the Spirit of Oppression's (119, replacing the
  lines before it while the base is above 0; a `MoraleAtMost`), the Angelic Alliance's join of
  the alignments (an `AlignmentGroup`: its factions count as one), a siege town's buildings
  (morale's building 5 +1, 0x660B48, and a Castle's 22 +2; luck's Rampart building 21 +2,
  0x660B40) and the owner's Grail line (§12.9; a `Building` source), both by a town record's
  index (the town lane's records, `packs/*/towns.toml`: the index to the building's original
  id); CLT-Q42 if answered by then; each line not derivable listed.
- **Acceptance:** as T2a2b2's; recorded: a siege town's buildings (the battle's map writes the
  town). **Depends on:** T3c1 (done). **Status:** ready.


- 2026-11-03 (E17 M6 T3c cut in two by the user, T3c1 done; the entry verbatim; T3c2 in the
  queue):

### E17 M6 T3c — morale's and luck's descriptions' carried lines

- **Scope:** the inbox's item of 2026-11-03 (CLT-Q40 carried by the battle lane, BAT-Q64,
  protocol v37; the entry in `QUEUE-ARCHIVE.md`): the stats dialog's morale and luck
  descriptions (design §12.4, its *not built*) from `Observation.morale_and_luck`,
  `Siege.bonuses` and `Stack.unclamped_morale`, `unclamped_luck`: the hero's lists
  (`hero_morale_description` 0x4DC320 and `hero_luck_description` 0x4DCAC0 read whole for their
  lines' texts: the Leadership or Luck level, the artifacts worn or in a combination, the visits
  by their once-marks, the owner's Grail building, the temporary rest), the Hourglass's and the
  Spirit of Oppression's suppression lines, the Angelic Alliance's join of the alignments, a
  siege town's buildings, and the header and picture by the unclamped value; each line not
  derivable listed. Until the adventure lane splits the battle bonuses by visit, they come as
  one temporary line each, as the original's rest. Recorded: the lines the battle's map can give
  (Leadership, Luck, a siege town's buildings).
- **Acceptance:** as T2a2b2's. **Depends on:** T2a2b2 (done), the carry (#685, merged).
  **Status:** ready; cut at its start with the user (likely: the hero's lists read and drawn,
  then the rest).

- 2026-11-03 (E17 M6 T3b2; the inbox entry turned into the task T3c, verbatim; T3b2 took the
  hero's own lists for the hero info window from it):

- 2026-11-03 (this lane, E17 M6 T2a2b2, CLT-Q40; carried by the battle lane, BAT-Q64,
  `cross-lane`): the carry is on the wire (protocol v37, public to seats and spectators):
  `Observation.morale_and_luck` (each side's hero's `MoraleLuck` changes, caps and alignment groups
  with a `Source`: a skill and its level, an artifact, the specialty, a visit's once-mark, a
  building, temporary), `Siege.bonuses` (the buildings by index with their changes) and
  `Stack.unclamped_morale`, `unclamped_luck`. Fill the descriptions' hero lists, the two
  artifacts' lines, the Angelic Alliance's join and a siege town's buildings, and the header and
  picture by the unclamped value (design §12.4). Until the adventure lane splits the battle
  bonuses by visit, they come as one temporary line each, as the original's rest. Your workspace's
  call sites were adapted to the new fields (the source on `BattleEffect`, defaults in the tests'
  wire literals).

- 2026-10-31 (E17 M6 T1 done; the entry verbatim):

### E17 M6 T1 — the answered slips (done 2026-10-31)

- **Scope:** the user's answers of 2026-10-31 (the lane's `QUESTIONS-ANSWERED.md`), each fixed in
  the client's profile with the original's kept for the comparisons only (`h3_render::Bugs`, as
  the cursor's x): CLT-Q26 (the book's last hovered item forgotten at each opening,
  `book::Memory`); CLT-Q27 (1) (a Faerie Dragon's aim given up posts the mouse, as the book's
  does) and (2) (the aim's start keeps the cursor's answer: a click before a move casts there);
  CLT-Q28 (1) (the box's text centred on the box) and (2) (the box framed in the acting side's
  player's colours), with the user's directive (the epic's "Decisions"): every text the client
  draws where the original sets it off its centre is centred (the readings of design §7.17 to
  §7.24 and §9 checked for such places); CLT-Q33 (no log line when nothing was summoned);
  CLT-Q35 (the result window says "fled" or "surrendered" only for the side that did).
- **Acceptance:** each fix tested in the client's profile and the original's kept for the
  comparisons (`drawn` all the same); `clients/scripts/check.sh` all gates; the design and M6's
  review list name the fixes.
- **Depends on:** nothing. **Status:** ready.

- 2026-10-31 (E17 M5 T8c, M5's closure at the user's checkpoint; verbatim): the queue's M5
  header, T8's entry, the inbox entries M6's tasks took (T2 the stack's info, T6 the rest) and
  the informational notices, moved out unchanged.

### E17 M5 — the queue's header at M5's end

Epic: `docs/epics/E17-clients.md` (M0 done 2026-10-12, M1 2026-10-13, M2 2026-10-14, M3
2026-10-23, M4 2026-10-27; ADR-0020, ADR-0024; the battle client since the split of 2026-10-23,
the adventure map client being the map-client lane's E24). M5 (live play) refined at M4's
closure (T3c, 2026-10-27, with the user): hot seat in-process first, then a game on a server;
its tasks below, in order (T1 and T2 done 2026-10-27, each cut in two by the user; T2b whole,
the user; T3 cut in two, T3a done 2026-10-27, T3b 2026-10-28; T4 cut in three, T4a done
2026-10-28, T4b cut in two, T4b1 done 2026-10-28, T4b2 and T4c1 done 2026-10-28, T4c2a and
T4c2b 2026-10-29: T4 done; T5 cut in two, T5a done 2026-10-29, T5b cut in two, T5b1 and T5b2
done 2026-10-29: T5 done; T6 cut in two by the user, T6a done 2026-10-30, T6b cut in two by
the user, T6b1 and T6b2 done 2026-10-30; T7 kept whole by the user, done 2026-10-30). The original files: stage them from the user's `original-data/install/` (one large file per staging call; the
user's computer must be online); never from the installation. The captures need the five DLLs,
`Heroes3.snd`, `H3ab_ahd.snd` and `H3ab_ahd.vid` too; the harness's runbook:
`clients/capture/README.md`. The client's gates: `clients/scripts/check.sh` (the full local
check; CI's job runs the same), the real-asset gate `cargo run -p h3-view -- gate` from
`clients/`. An asset fix gets its mechanism (ADR-0020 point 6: the entry's flag, the
`faithful`/`fixed` profile) with the first one a task finds.

### E17 M5 T8 — M5's closure (done 2026-10-31, T8c)

- **Scope:** a fresh reviewer, the battle recipes' starting bonus set to Gold (M4's review item
  15), every capture afresh, the review list, the next milestone refined with the user (the
  ladders, the pause, the server browser; HD; graphics options).
- **The user's cut (2026-10-30):** concurrent sub-agents, two capturing (one per harness
  instance: the runbook's "Two instances") and one reviewing or fixing, each given a fraction
  of about half a context; the runner keeps only their reports. **T8a done** (log, "M5 T8a"):
  the two instances, the bonus Gold, reviewers R1 (network, accounts), R3 (the play core), R4
  (the spell book) and their fixes; all 91 stills and sequences 0 to 27 as recorded.
- **T8b cut in two by the user (2026-10-30, the agent's proposal):** **T8b1** the captures
  (sequences 28 to 64), R7 and R2 (in two: R2a the decoder, R2b the clip) with their fixers;
  **T8b2** R4b, R5a to R5c, R6, the leftovers, then `drawn` and `check.sh`. **T8b1 done** (log,
  "M5 T8b1": sequences 28 to 64 as recorded, R7, R2a and R2b fixed; review items 73 to 77).
- **T8b2 done** (2026-10-31, log "M5 T8b2"; the user's fractions: three concurrent sub-agents,
  the runner the leftovers): reviewers R4b, R5a to R5c, R6 and their fixers; the leftovers (an
  entered stack carries the playhead, the name lookup's wait, `RecordError::Line` in numbers, the
  sequences tool naming a bad clip, `bink-reference.py` on a refused packet, R7-8 the Gold bonus
  read in memory); `drawn` and `check.sh`; review items 78 to 86, CLT-Q35.
- **T8c (the user's checkpoint):** the review list (items 1 to 86), CLT-Q35, the next
  milestone refined; M5 done.
- **Acceptance:** the review list accepted; every capture as recorded or its difference listed;
  the next milestone's tasks in this queue; M5 done.
- **Depends on:** T1 to T7 (done 2026-10-30). **Status:** T8a, T8b1 and T8b2 done; T8c ready (the user).

### NOTES.md entries done by M6 T6b1 (2026-11-07; `NOTES.md`, "Battlefields and boats")

- 2026-10-15 (E17 M3 T2): **when CLT-Q5 is answered** (boats in `h3-map`'s writer): the deck and
  the two boats as capture-set situations (`h3-view situation`'s `special` refuses them now: a
  hero in a boat against one on land, or both heroes in boats), their pictures already drawn.

### Inbox entries taken by M6 (2026-10-31)

- 2026-10-29 (the user, at E17 M5 T5b1's start; for a free slot or M5's closure): **the
  Demons' dissolve, re-investigated.** The slip of T5a (the saved picture is the back buffer)
  is the rock clipped out; the Imps' corpse gone before the fade-out's end is not part of it in
  the user's view. Find where the corpse leaves the original's picture and model it if it is not
  the slip (design §7.25, "To re-investigate"; `seq-demons.toml`'s 8 dissolve frames).

- 2026-10-28 (the adventure lane, E08 T67; CLT-Q5's answer, decision 158): **boats in
  `h3_map::writer`.** An empty boat is a `Plain` object of class 8 (`PLAIN_OBJECTS`, the
  `Objects.txt` templates `AVXboat0` to `AVXboat2`, the type as subclass): as a hero's template
  it stands one column west of its position, on water, blocked and visited there. A map cannot
  put a hero in a boat (the original makes no boat for a hero placed on water, survey
  `docs/rules/surveys/E08-sea.md` §2, §5; batch E8's E8-5 tests it): a situation with a hero in a
  boat places the hero on the coast beside an empty boat and boards it in the game (a click on
  the boat), then the deck (a hero on land attacks it from the shore) or the two boats (both
  boarded, side by side). Batch E8b's E8-11 and E8-12 (`experiments/specs/E08-batch-8b.toml`) are
  the two setups. Also new: Flotsam, the Sea Chest, the Survivor and holes as `Plain` objects, an
  Ocean Bottle or sign with its text (`RewardBody::Sign`), the spec's `[[map.bottle]]` and an
  opponent monster's own `at` (any terrain).

- 2026-10-28 (E17 M5 T4b2b, CLT-Q30, the user: listed now, exact later): **`h3-bink` to decode
  as the game's `BINKW32.DLL` (Bink 0.5b).** Its inverse DCT is x87 single-precision floating
  point (`ExpandPlane`'s 8 × 8 transform: √2, 1.848, 2.613 and 1.082 as floats, float
  quantizers), FFmpeg's integer, so five clips differ by one level on a few pixels (`rtstart`,
  `rtloop`, `Surrender`, `defendloop`; `Defendall` up to 388 a frame; design §7.21). The task:
  the DLL's transform and quantization in integer arithmetic (no floats in the client: exact
  binary32 emulation or a proof that a fixed-point form rounds alike on every input the
  bundles allow), checked against the DLL's own planes (`binkprobe frames <vid> <clip> 7`:
  YUY2) on the ten clips and the synthetic samples; FFmpeg's check (`bink-check.sh`) then
  becomes the DLL's. Then the captures' listed DCT pixels (`result-retreat` 2 700 and 3 000,
  `defended`) go.

- 2026-10-28 (the map client lane, E24 M3 T2a; additive, nothing of yours drawn changed): **the
  dialog items' drawers in the common crate.** `h3_ui::paint::draw` draws each `h3_ui::ui::Draw`
  (picture, button, text, label, darken) as `h3_view::compose` does (ported from it, the same
  pixels) from any batch implementing `h3_ui::paint::Assets` (`sprite`, `picture`, `palette`,
  `font`), and `h3_ui::paint::names` gives the assets a draw needs; the map viewer draws its
  adventure dialog with it. M3's exit wants `h3-ui` the only copy of what both viewers share:
  your compositor's `Draw::Dialog` arms and `batch.rs`'s names switched to it (your `Batch`
  implementing `Assets`) remove your copy. Also additive: `Dialog::draws` gives a button the
  player's colours (`Shown::players`) and a DEF (`Shown::defs`) by its id, which no battle
  dialog sets (`h3drive.py drawn` all the same).

- 2026-10-27 (E17 M5 T2a, the click read; for M5 T4 or later): **the stack's and the turret's
  info by the mouse** are not built: a left press on a stack to look at (kind 5) opens its stats
  dialog (`stack_stats_dialog` 0x468860, modal), on a turret (22) its info (0x47A380); the right
  button pressed over the battlefield shows a popup while held (0x4769C0, not read), over a
  control its help (0x472A50). `mouse::click` gives no command there (design §9.11).

- 2026-10-24 (the map-client lane, E24 M1 T6b's reviewer, R2-15; for you to take or leave):
  **`h3-view`'s `tests/purity.rs` has three holes** its copy in `h3-map-view` had: `sources` skips
  every folder named `bin` at any depth (only `src/bin/` is the adapter), a denied lint relaxed
  through its group (`#[allow(clippy::restriction)]`) or by an attribute over several lines is not
  seen, and the required header is matched as a substring (a commented-out `// #![no_std]`
  passes). `clients/crates/h3-map-view/tests/purity.rs` (E24 M1 T6b) has the fixes:
  `walk` from `src` leaving out `src/bin` only, `attributes` joining an attribute's lines, the
  `GROUPS` list, the header as a line of its own.

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R4-2: `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s
  boxes, the log's text: digests recorded again from `master`'s viewer first, the runbook's
  rule). R4-9: a stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an
  artifact over almost anything; the counts stay exact). (R2-7, R2-8 with R1-3 and R3-1 went into
  M5 T1, T6 and T5 at M4 T3c; the entry's full text in `QUEUE-ARCHIVE.md`.)

### Informational notices (moved 2026-10-31)

- 2026-10-30 (the adventure lane, E08 T79a; for information, nothing to do): the harness's
  reads and steps for batch E9a (`clients/capture/README.md`, "Mines, Lighthouses and Shipyards").
  Behaviour that changed for every recipe: `h3lod.identify` also knows `MINEEVNT.TXT` (a mine's
  flag message, `MINEEVNT:<n>`; it was named GENRLTXT:4, its picture's caption) and lets a
  record's title braces be missing; a walk records the cursor before its first click (`cursor`);
  a battle's record keeps each side's owner and controllers (`sides`); the boat dialog (vtable
  0x643908) is a screen a step's `actions` play, cancelled when left open. New: `mine:x,y`,
  `tile:x,y`, `screens`, `opening:sides`, `opening:stacks:<side>`, `opening:slots:<side>`, `stops`,
  `battle = "auto again"`, `row:<column>#2`; `h3mem.mines`, `tile_word`, `battle_sides`,
  `auto_combat_on`. The harness check passed.

- 2026-10-30 (the adventure lane, E08 T79b; for information, nothing to do): the harness's
  reads for batch E9b (`clients/capture/README.md`, "The day's start"). Behaviour that changed for
  every recipe: `object:x,y` and `word:x,y` of an object a hero stands on read it through the
  hero's record (+0xC, +0x14; they read the hero's class 34 and index before), `fill` and
  `compare` key a row label repeated in a table `<label>#<n>` (its second row stayed empty before),
  `message_record` names a message whose text is one of the spec's own `the map's text "..."`.
  New: `income[:<colour>]`, `gardens[:<colour>]`, `growth:x,y`, `gain[:<colour>]`, the weekly
  objects' `object:` texts (Windmill, Water Wheel, Mystical Garden), an end-turn step's `record`
  and `note` (the new turn's messages: `end_turn(game, mem, events)`, `dismiss(..., mem, events)`,
  both optional), a recipe's `gold = true`; `h3mem.player`'s `income` and `gardens`,
  `h3mem.monster`'s `sixteenths` and `word`. The harness check passed.

- 2026-10-30 (the map client lane, E24 M3 T6a2b, the user's consent; for your information,
  nothing to do): `h3-view`'s test `the_committed_catalogue_parses` lists SEERHUT among the
  tables the catalogue's keys read: the map hint's quest texts (`map.quest.<type>`, a form per
  SeerHut.txt column) are keys, which every client reads. The catalogue also has 33 more `map.`
  keys and 13 optional families only the map viewer lists (`clients/text/keys.toml`).

- 2026-10-29 (the adventure lane, E08 T68b; for information, nothing to do): the harness's
  reads and steps for batch E8b (`clients/capture/README.md`, "Boats and the sea", since T68b).
  Behaviour that changed for every recipe: `h3lod.identify` also knows `RANDSIGN.TXT` (a sign's
  or Ocean Bottle's default text, `RANDSIGN:<n>`) and lets a record's doubled quote match one or
  two (ADVEVENT 126 was not identified); a message one of the map's message records holds reads
  `the map's text "..."` before any table is tried (GENRLTXT 16, `%s the %s`, took it);
  `object:x,y` reads `none (class 3: the coast)` for a coastal tile without an object; `select`'s
  `controls` count only tiles that hold an object. New: `object:x,y` for the Sea Chest, the Survivor and the Ocean
  Bottle (an artifact chest and a Survivor hold an artifact for `require` and `select`), `select`
  by `is`, `count` and `then`, `h3mem.open_tiles(…, water=True)` (a hero in a boat approaches over
  water), `h3mem.messages`, table rows filled step by step (`row:<first column>`), a grid step's
  `note` at every tile, and the reads `battle:tile`, `boat_record:x,y`, `treasury_changed`,
  `text:<literal>`. The harness check passed.

- 2026-10-28 (the adventure lane, E08 T68a; for information, nothing to do): the harness's
  reads and steps for batch E8a (`clients/capture/README.md`, "Boats and the sea"). Behaviour that
  changed for every recipe: `h3mem.path` (the `marks`, `mark` and `path_tiles` reads) is empty
  while the map shows no path (adventure manager +0x4C, as `path_arrows`): a click that found no
  path left the last path's arrows, which it read before. New: the battle's ship flag in
  `battlefield`, `h3mem.obstacle_hexes`; `h3adv`'s `boat`, `boat[<hero>]`, `boat:x,y`,
  `boats:used`, `drawn`, `opening:field`, `battlefield:ship`, `blocked`, a Flotsam's `object:x,y`,
  and the steps `key`, `again_if`, `only_if = "aboard"` and `dump`. The harness check passed.

- 2026-10-28 (the map client lane, E24 M3 T1b; **your window's keys changed**; the user: every
  key through the table, no cut): **the battle window's keys go through `h3_ui::keys`** (#548,
  `cross-lane`, `map-client/e24-m3-t1b`, named in the map client lane's HANDOFF until it merges). The new `h3_view::keys` holds the battle's commands with their names and the
  original's keys by **context** (`Context::of`: your M5 T4b1's result window, else a message box,
  else the book, else an aim, else the battle): the window's own (`h3_ui::keys::Window`, the map's: F2 music, F3 the creatures'
  sounds, F5 pause, F6 a frame on, F7 restart, F8 vsync, F9 the cursor, F11 fullscreen, Esc
  quit; P, →, Home, M, E, V and H gone), `log-up`/`log-down` (↑ ↓), `dragon-spell` (F),
  the panel's seven buttons (`BUTTONS`: S, R, O, A, C, W, D and Space), the book's
  (`book::Key`: Esc and Enter close, A, C, ← → ↑ ↓) and the box's (`message::Key`: Enter OK,
  Esc Cancel), `aim-cancel` (Esc, the right button), `result-ok` (Enter and Escape, 0x4715C3;
  merged with #547: `result::window`'s OK and the stills' `one_button` lost their `keys`). The user's `keys.toml` (`--keys`, else the
  working directory) has a `[battle]` table (the map's `[map]`). API changes: `Book::key` and
  `book::Open::key` take a `book::Key`; `message::Open::key` a `message::Key` and the key's name
  (its release still by name); `h3_ui::ui::Button` lost `keys` and `Dialog::button_for_key`
  (the layout's panel `keys` gone: the table's defaults are their only copy); the stills' `key:`
  actions go through `Keys::original()` by context; `h3_sdl::Key::ALL` gained the missing
  `Return`. Behaviour changed: the book takes every key while open (the panel's letters and F
  reached the battle under it); unattended, Esc quits whatever is open and the right button no
  longer gives an aim up; a chord with Ctrl held gives nothing unless bound. `viewer_key` and
  `book_key` are gone. The 2026-10-26 entry below (custom key bindings) is done by it; your
  M5 T3b's F is the table's `dragon-spell`. A branch of yours merges `master` in after it merges.

- 2026-10-28 (the map client lane, E24 M3 T0; **decided by the user for both lanes**; the
  proposal of 2026-10-27 as it stood: `docs/lanes/map-client/QUEUE-ARCHIVE.md`): **the common crate
  `h3-ui`** (`clients/crates/h3-ui`, common to both client lanes as the foundation is: a
  `shared/` pull request for `lanes.toml` first) takes `h3-view`'s `clock`, `cursor` with
  `batch::Sprite`, `ui` (with draw items of its own instead of `scene::Draw`'s) and `text`, and
  `h3-map-view`'s `keys` (generic over a viewer's commands) and `music`; `layout` and `mouse`
  stay in `h3-view` (battle-only). The map client lane moves it as **E24 M3 T1a** in a
  `cross-lane` pull request (the user's consent): `h3-view`'s imports change, nothing it draws
  or plays. Then **E24 M3 T1b** takes `h3-view window`'s keys through the table (the 2026-10-26
  entry below), **both windows' own keys the map's** (the user: F2 music, F3 effects, F5 pause,
  F6 the next frame, F7 restart, F8 vsync, F9 the cursor's kind, F11 fullscreen, Esc quit; P,
  →, Home, M, E, F, V and H gone), so M5 T3b's F key is free. Starting M5 T3b before T1a and
  T1b merge means merging `master` in after them (the imports); the map client lane tells you
  here when each merges.

- 2026-10-26 (the map client lane, E24 M2 T1b; **for you, at M3 T0**): **custom key bindings**
  (the user, 2026-10-26: keys hard-coded so far are to be phased into bindings the player can
  change). The map client's window (E24 M2 T4) takes every key through a table of actions with
  the original's keys as defaults and a user file overriding them; at E24 M3 T0 that table moves
  into the common crate both lanes share (with `clock`, `cursor`, `ui`, …), and `h3-view
  window`'s keys (now matched in its event loop, `window.rs`) go through it too. Nothing to do
  before M3 T0's proposal; `h3-sdl`'s `Key` grows additively meanwhile (a common path).

- 2026-10-30 (E17 M5 T7 done, kept whole by the user; the entry verbatim):

### E17 M5 T7 — accounts

- **Scope:** the account's key from the login and the password (argon2id with the server lane's
  parameters, the seed as Ed25519's secret key; the test vector in `NOTES.md`, "The live
  client"), derived once per device and cached (never the password); the login signed only for
  the address dialed, a registration's display name; seats by account (joined without a token,
  re-taken after a drop); `Rekey` after a password change; the goodbyes `Rekeyed`,
  `CommandLimit`, `Abandoned`. Its dependencies (argon2, Ed25519): asked of the user at the task.
- **Acceptance:** the test vector reproduced; a battle played logged in on a local `h3-server`,
  the seat re-taken after a drop; the Definition of Done.
- **Depends on:** T6b2 (done 2026-10-30); the user's choice of the dependencies. **Status:**
  ready (T6a's `remote` keeps the welcome's challenge out until the login needs it).

- 2026-10-30 (E17 M5 T6b2 done: T6 done; the entry verbatim):

### E17 M5 T6b2 — the record that opens mid-battle

- **Scope:** a record that opens mid-battle (R2-8: the snapshot's dead stacks as corpses, a raise
  of one drawn, the box by the timeline's living set; M4 T3a's R1-3: the moat's strip by the
  fixed moat rows, 0x63BCE8 and 0x63BCF4, not the record's moat) for a join mid-battle and a
  re-join after a server restart (the numbering afresh: a new timeline); the window re-dialing
  until a restarted server answers. The detail: `QUEUE-ARCHIVE.md`, 2026-10-23 (T9b). Where
  T6b1 stops (design §11.2): `online::Online::take` refuses a fetched record that does not extend
  the one taken; `Ground::load` keeps the record's first snapshot as the opening.
- **Acceptance:** a join mid-battle; a server restart and a re-join; the mid-battle record drawn
  against the original where it has a reference; the Definition of Done.
- **Depends on:** T6b1 (done 2026-10-30); the original data staged. **Status:** ready.

- 2026-10-29 (E17 M5 T5b2 done: T5 done; the entry verbatim):

### E17 M5 T5 — what a played battle shows

Cut in two by the user (2026-10-29): T5a the dissolves and the created stacks (with the engine's
event for a stack without a place, `cross-lane`; done 2026-10-29), T5b the panel by each turn's
controller, the sounds' waits and the palette effects, cut in two again by the user (2026-10-29,
at its start): T5b1 the palette effects (done 2026-10-29), T5b2 the panel and the sounds' waits.
The detail: `QUEUE-ARCHIVE.md`, 2026-10-22 (T7c-3).

#### E17 M5 T5b2 — the panel by the turn's controller, the sounds' waits (done 2026-10-29, kept whole by the user)

- **Scope:** the panel's side by each turn's controller (R3-1: Hypnotize, a created stack); the
  battle waiting for the heal's sound and the other effects' (the Pit Lord's `Resurect.wav`,
  0x5A754E, among them). Recorded against the original.
- **Acceptance:** the new sequences as recorded, or each difference listed; the Definition of
  Done.
- **Depends on:** T5a (done). **Status:** ready.

- 2026-10-29 (E17 M5 T5b1 done; the entry verbatim):

#### E17 M5 T5b1 — the palette effects (done 2026-10-29)

- **Scope:** `draw_battle_stack`'s palette effects (0x43E280 to 0x43E4C7; the user,
  2026-10-29): a clone recoloured (0x522B50 with 0.67, 1, 2, 2 on palette entries 10 to 255: RGB
  to HSV in single precision, the hue set, saturation and brightness moved toward 1, back to
  RGB; `clone`'s frames differ by it alone, design §7.25), Bloodlust's glow (the same routine)
  and Stone Gaze's (0x522A10), Stone Gaze's petrified look (0x522D50). Recorded against the
  original.
- **Acceptance:** the new sequences as recorded, or each difference listed; `clone`'s colours
  exact; the Definition of Done.
- **Depends on:** T5a (done). **Status:** done (`client/e17-m5-t5b1`).

- 2026-10-28 (E17 M5 T4b1: `master` merged in after #545, the imports followed; the entry verbatim):

- 2026-10-28 (the map client lane, E24 M3 T1a; **your imports changed**): **`h3-ui` built**
  (`clients/crates/h3-ui`, common to both client lanes since #541's `lanes.toml`; the move in a
  `cross-lane` pull request of the map client lane's, named in its HANDOFF until it merges).
  `h3-view`'s `clock`, `cursor`, `ui` and `text` are `h3_ui::clock`, `cursor`, `ui` and `text`
  (`h3_view::{clock, cursor, ui, text}` gone; the binary's own `text` adapter stays);
  `batch::Sprite` is `h3_ui::sprite::Sprite`; the scene's five dialog variants (`Picture`,
  `Button`, `Text`, `Label`, `Darken`) are one, `Draw::Dialog(h3_ui::ui::Draw)`, with
  `PlayerColours` in `h3_ui::ui` (`Dialog::draws` gives `ui::Draw`s; `scene::panel` and
  `Book::draws` and the binary's message boxes wrap them; `compose` and `batch::names` match
  them inside `Draw::Dialog`; your M5 T4a's `size` and `player` merged into them, `message` on
  `h3_ui::{ui, text}`);
  `tests/purity.rs`'s allowlist for `h3-view` gains `h3-ui` (`serde_json` is now the binary's
  only, left in the list). Nothing drawn or played changed (`h3drive.py drawn` and `h3map.py
  drawn` all the same). `h3-ui`'s own `tests/purity.rs` has R2-15's fixes (the 2026-10-24 entry
  stands for `h3-view`'s). A branch of yours from before it (M5 T3b) merges `master` in after it
  merges: the conflicts are these paths. T1b (the battle window's keys through `h3_ui::keys`)
  follows.

- 2026-10-27 (M4 T3c: taken by R2-7 into M5 T1, R2-8 with M4 T3a's R1-3 into T6, R3-1 into T5; R4-2 and R4-9 stay in the inbox; the entry verbatim):

- 2026-10-23 (T9b, the reviewers; the epic log's "M3 T9b", the accepted review list): queued
  findings. R2-7: a timeline that steps forward (`Moment::at`, `Heroes::posed` and `Timeline::at`
  rescan the battle for every frame: 2.3 ms a frame over 82 seconds, `play`; for live play's long
  games). R2-8: a record that opens mid-battle (the snapshot's dead stacks as corpses, a raise of
  one drawn; the box by the timeline's living set, not the first snapshot's `alive`; with it
  M4 T3a's R1-3: the moat's strip over a stack by the fixed moat rows, 0x63BCE8 and 0x63BCF4,
  not the record's moat, which leaves out the bridges while the opening's gate is open). R4-2:
  `drawn`'s digest of what the masks depend on besides the pixels (`ours.txt`'s boxes, the log's
  text: digests recorded again from `master`'s viewer first, the runbook's rule). R4-9: a
  stricter shadow chain in `h3cap.redraws` (a near-black pixel counts as an artifact over almost
  anything; the counts stay exact). R3-1 goes with the created stacks below.

- 2026-10-27 (M4 T3c: taken by M5 T3; the entry verbatim):

- 2026-10-21 (T7b-1): **a hero's aim.** Between the book's place and the target's click the
  cursor is the spell's (`crspell.DEF`, animated: not read; the viewer leaves it as it was) and the
  status line says "Cast %s on %s" (GENRLTXT 28, built *provisional*, not forced; 27 elsewhere);
  `enter_player_cast` 0x59EC50's loop is not read (which targets it offers, the cursor's frames):
  with the mouse's model when a recording aims a hero's area spell at a hex (T7c-1's are the
  Faerie Dragon's clicks), and for live play. And `cast`'s 71
  pixels at 0: the original redraws the battlefield over a rock's top (obstacle 22) for the one
  frame of the click (0x479038's path, not read).

- 2026-10-27 (M4 T3c: taken by M5 T4 (the volumes to `NOTES.md`, "Options and layouts"; `remote_turn` to T6); the entry verbatim):

- 2026-10-21 (T8): the battle's result window (`battle_result_dialog` 0x4702D0, not read beyond
  its tune: the heroes, the texts, the casualties, OK) and the whole screen darkened under it
  (0x44E5F0 from 0x47782C): `end`'s last frame differs by them (its counts). The volume settings
  (0x5996C0) and a track resuming where it stopped (0x59AA82) with the client's options and a
  second battle (later); the network's `remote_turn` in the end's choice with live play.

- 2026-10-27 (M4 T3c: taken by M5 T5; the entry verbatim):

- 2026-10-22 (T7c-3): **created stacks are not drawn.** `StackCreated` (Summon Elemental, Clone)
  and a Pit Lord's `RoseFromCorpse` add a stack the record's first snapshot lacks, and the scene
  draws only the snapshot's stacks (a wire `Stack` each): the viewer needs the new stack's
  numbers (from the ruleset, or a later snapshot in the record), its record index and the
  dissolve that shows it (`create_battle_stack`, 75 ms steps, the blend not read; T9d built a
  dissolve's time for Sacrifice's victim, `anim::GONE`, `DISSOLVE_STEP`, and the frame check of
  `sacrifice` lists its blend: `screen_dissolve` 0x602DC0, read for both); with them the
  message when no hex is free, a clone's `Vanished`, Teleport's dissolve (moved at once now) and
  Sacrifice's victim leaving no corpse; and the panel's side by each turn's
  controller (T9b, R3-1: `scene::controller` takes the opening's, so Hypnotize and a created stack
  are not followed). The battle waits for the heal's sound (T3b) and the other
  effects' (T7b-2b, T7c-3). NOTES, "The battle screen".

- 2026-10-27 (M4 T3c: taken by M5 T4; the entry verbatim):

- 2026-10-20 (T6a): the tactics phase's panel (0x46C050, not read) and Auto Combat switched on
  (its button selected, the panel dimmed while stacks play themselves) wait for a record with a
  tactics phase and for live play. Design §9.7.

- 2026-10-27 (M4 T3c: taken by M4, done 2026-10-27; the entry verbatim):

- Sieges (the battle lane, 2026-10-14 to 16): the turrets as stacks (drawn as the town's
  shooters since T1b), the Catapult, the gate opening before the step. NOTES, "Sieges"; M4 T2b
  to T2c-2 took the shots, the gate, the moat, the mines, Earthquake and the mouse (design
  §10.2).

- 2026-10-27 (M4 T3c: taken by M5 T6 and T7; the ladders, the pause and the server browser after M5; the entry verbatim):

- The live client (the server lane, 2026-10-09 to 13): the record file, the content check, a
  server restart, accounts, ladders, the server browser. NOTES, "The live client".

- 2026-10-26 (the adventure lane, E08 T57; for information, nothing to do): the harness's
  reads and steps for batch E7 (`clients/capture/README.md`, "Rewards on the map"). Behaviour that
  changed for every recipe: `before:<observation>` is now read right before the step (any
  observation, tiles too; it was the step's snapshot), a message box whose OK waits for one of two
  pictures is answered by `choice` (it stopped the batch before), and each message closed with its
  OK keeps its pictures (`message_pictures`). New: `h3mem.object_at`, `universities`,
  `dialog_pictures`, `open_tiles`, `map_kind`, a hero's spell points, bonuses and status flags, a
  stack's army slot; `h3adv`'s `each`/`select` grids, list records and `+` fields, `once`,
  `days`, `near` and `[[require]]`. The harness check gained Win's chest and Warrior's Tomb.

- 2026-10-26 (T2c-2): **the status line at a human turn with the mouse parked** is kept in hot
  seat and emptied with one human side, built as found (*provisional*; design §10.2, "Recorded
  (M4 T2c-2)"). The dispatch is read (an item writes its hint only on entering, `hovered_item`
  0x6AAC68); open: the item under the park point and what makes it entered again (`aim`'s line
  emptied at 8 175, `hit`'s kept). A capture reading 0x6AAC68 and that item's id per frame
  would settle it (T3's captures or live play). Done at M4 T3b-1 (2026-10-27,
  design §10.2 "Recorded afresh").

- 2026-10-23 (the map-client lane, E24 T0; for later, nothing to do now): **what the map client
  will need of `h3-view`.** E24 M1 (stills of the map view) builds only on the common foundation;
  M2's window will want `clock` and `sound`, M3's panels `ui`, `text` and `layout`, in a common
  crate. When M2 or M3 is refined, the map-client lane proposes the move here first (a new common
  crate is a `lanes.toml` change, shared); your M4 waits for none of it.

- 2026-10-23 (the adventure lane, E08 T47a; for information, nothing to do): the harness's
  clicks and reads for batch E6 (`clients/capture/README.md`, "Screens and their items"): the
  screen showing is the topmost dialog's kind by its vtable (`h3mem.dialog`), and a click finds
  its item by id among the dialog's items in memory, which now carry their boxes, states and texts
  (`h3mem.dialog_items`; `dialog_text` shares `item_text`); the hero screen, exchange, army,
  creature and split dialogs' items by name (`h3adv.SCREEN_ITEMS`); a step's `screen = "hero"`
  and `actions`; a battle's tactics phase ended at once in `fight` (its message's OK, then Start).
  The harness check gained Kit and Full (Blue's eight heroes): passed under Wine.

- 2026-10-21 to 22 (the adventure lane, E08 T32b1 to T37b; for information, nothing to do): the
  harness's reads and steps for batches E4 and E5 (moved to NOTES, "The adventure lane's
  harness", at T9c; the harness is common since the split, 2026-10-23).

- 2026-10-20 (the adventure lane, E08 T22a2b): its three recipes, its auto combat's change, for
  information (NOTES, "The adventure lane's harness"; the winner of −1 fixed in E17 M3 T6c-2).

