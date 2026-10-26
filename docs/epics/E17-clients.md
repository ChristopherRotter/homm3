# E17 — Clients and asset pipeline

Status: **in progress**: M0 done (2026-10-12); M1 done (2026-10-13: T1, T2, T3 and T6 on 2026-10-12, T5, T4, T7 and T8 on 2026-10-13); M2 done (2026-10-14: T1, T3, T4a, T6 and T4b-1 on 2026-10-13, T4b-2a, T2, T5a, T5b and T5c on 2026-10-14); M3 done (2026-10-23: T1a–T1c, T2, T3a, T3b, T4a, T4b-1, T4b-2a, T4b-2b-1, T4b-2b-2, T5a, T5b-1, T5b-2a, T5b-2b, T6a, T6b, T6c-1, T6c-2, T7a-1, T7a-2a, T7a-2b, T8, T7b-1, T7b-2a, T7b-2b-1, T7b-2b-2a, T7b-2b-2b, T7b-2b-3, T7c-1, T7c-2, T7c-3, T7d, T9a, T9b-1, T9b, T9c, T9d); M4 refined (2026-10-23, T0), T1a, T1b, T2a and T2b done (2026-10-23), T2c-1 (2026-10-24); the adventure lane's harness H1 and H2 done (2026-10-19). Refined 2026-10-12 (E17a, the asset notes,
done the same day). Decisions: [ADR-0020](../adr/0020-clients.md) (accepted at the M0 checkpoint,
point 7 rewritten there). Working detail (code readings, formats, layouts): `docs/epics/E17-design.md`,
created by the first task that has detail to record; done tasks in full:
[`E17-log.md`](E17-log.md). Lane: client (`docs/lanes/client/HANDOFF.md`; the user's asset notes
are there). Next: M4 T2c-2 (sieges: Earthquake, the shots and the mouse recorded), M5 live play after M4
(`docs/lanes/client/QUEUE.md`). Since T9c (2026-10-23) this epic
is the battle client's; the adventure map client is E24, the map-client lane's.

## Goal

Clients that show the engine's games with the original's own assets, truthful to the original
down to the pixel and the frame: first a battle replay viewer, then sieges and playable battles.
Everything with graphics, sound, input or a window lives in `clients/`, outside the core. The
adventure map, the full game record/replay viewer and whole games played are E24's (the
map-client lane, since 2026-10-23), on the same foundation.

## Decisions (user, 2026-10-12; 2026-10-13 where dated)

- **Rust**, confirmed; the `clients/` workspace as Phase 0 laid it out.
- **A software compositor** into a frame in the original's screen format; render-to-PNG writes
  that same frame. HD is planned for (frame size and layouts are data) but comes later.
- **The window, input and audio library is chosen at M2 after an evaluation**, on technical
  grounds only. `winit` and `cpal` are Apache-2.0-only, which ADR-0009 excludes; if they win the
  evaluation, the client may move to GPLv3 (its own `deny.toml`), the user deciding at M2.
  **Chosen 2026-10-14 (T2): SDL 3** through `sdl3-sys`, built from source and linked statically;
  no licence step ([ADR-0024](../adr/0024-client-window-audio.md), design §8).
- **Asset tables per expansion, committed** (`clients/assets/{base,ab,sod}.toml`); no reading of
  the executable at run time.
- **Reference captures:** automatic (2026-10-13: the user cannot take screenshots for exhaustive
  diffs): the agent runs a copy of the original from `original-data/` unattended in the cloud
  sandbox under Wine and dumps the 16-bit screen (`clients/capture/`, design §6); this replaces
  the earlier plan of computer use on the user's machine. Never from the installation: running
  the game writes into its folder (saves, settings, DDrawCompat's log).
- **M1's exit through the automated diff** (2026-10-13): the stills are compared with the
  original by the first automated diff, which passes or lists its differences, not by the user's
  eye.
- **First milestone: M0 + M1** as below: headless, independent of the battle lane, starting now.
- **Loading at checkpoints** (M0 checkpoint): at start only the archives' directories; at each
  checkpoint (the client opens, a game starts, a battle starts, …) one batch of every asset the
  coming screens need, read by offset; nothing loads while a screen is shown. The bytes come from a
  table-only reader in `h3-formats` (CLT-Q2, the battle lane), chosen over memory-mapped files and
  whole-archive reads; until it lands the adapter reads whole archives (ADR-0020 point 7).
- **Text through a string catalogue** (2026-10-13): every string the client shows is an entry of
  a key → string catalogue (nested JSON or similar), easy to translate; language packs swap. The
  original's text stays uncommitted (ADR-0009): its language comes from the user's own files
  through a committed catalogue of keys. **The packs are nested JSON** with i18next's conventions
  (2026-10-19, M3 T4b-1, over TOML and Fluent), and **a translation of the original's text is
  never committed**: a player may load a pack of their own, which the project neither ships nor
  hosts (CLT-Q7, 2026-10-19). An ADR-0020 amendment is drafted in `docs/lanes/client/adr-drafts/`.
- **Graphics options** (2026-10-13), eventually: per effect the original's way or a modern one
  (shadows first); `faithful` is the default and the reference. The draw list names what is
  drawn, the compositor decides how (QUEUE inbox).
- **The original's redraw artifacts are not copied** (2026-10-14, M3 T1a): the original redraws
  by dirty rectangles (design §7.11), which darkens shadows again and leaves stale count boxes;
  the viewer keeps drawing each frame whole in every profile, and the frame check classifies
  those pixels apart (a pixel that is ours with one to five more shadows) while any other
  difference still fails (M3 T1c, done). An ADR-0020 point 9 amendment is drafted in
  `docs/lanes/client/adr-drafts/` for the next checkpoint.
- **The original's drawing bugs are fixed, and reproduced only for the comparisons** (2026-10-15,
  M3 T4a): a bug in the original's drawing code (the first: the text drawer loses the glyph at a
  long word's break) is fixed in what the client draws; the comparisons with the original's
  captures (the capture set, the frame check) draw it as the original does, so that they stay
  exact. A development switch (`h3_render::Bugs`), neither a player's option nor a quirk
  profile. Goes into the ADR-0020 point 9 amendment draft. The second (2026-10-20, M3 T6c-1):
  the software cursor drawn at an even x (0x50CE04, design §9.9). The third to fifth
  (2026-10-22, M3 T7c-1, CLT-Q8: "dev-only for the captures, fixed otherwise"; fixed in T7d,
  the switch reaching the timeline: `anim::Stage::bugs`): a creature caster's group by the
  angle's wrong sign, a caster its spell missed kept on its casting frame until the whole
  redraw, the target's highlight kept through a creature's frames.
- **Every oddity of the original is an open question** (the user, 2026-10-22: so that nothing is
  forgotten): a drawing quirk or bug a reading or a recording finds becomes a `CLT-Q<n>` in
  `docs/OPEN-QUESTIONS.md` in the same task (copy it, fix it with the original's only for the
  comparisons, or a player's option; a recommendation), and the review list names it.
- **The client lane split** (2026-10-23, M3 T9c, CLT-Q18): with the rendering basics done, the
  user split the lane: this lane (`client`) keeps the battle client, M3 closing with T9d, then
  **M4 sieges, then M5 live play**; a new lane, `map-client` (ids `MCL`), builds the adventure map
  client in parallel as **E24**. The client foundation (`h3-assets`, `h3-render`, `h3-sdl`, the
  workspace's files and gates, the string catalogue) and the capture harness became common paths
  (`lanes.toml`): either client lane changes them in its own pull request, an additive change
  freely, a change in behaviour noted in the other's inbox; the adventure lane changes the
  harness without the `cross-lane` label.
- **The natural obstacles' types first** (2026-10-13, CLT-Q3): the engine dropped them at
  generation; a `cross-lane` change carries them to the wire before T7 draws obstacles.

## The ideas in plain words

- **Indexed sprites, one frame.** The archives hold sprites as 8-bit indices into a 256-colour
  palette; some indices are not colours but instructions (transparent, shadow, selection outline,
  player colour). The compositor draws the sprites into one frame the way the executable does,
  then hands the frame to the window or to a PNG file. What an agent sees in a PNG is what a
  player sees.
- **Asset tables.** The engine names things by key (`base:pikeman`); the archives by file name
  (`CPKMAN.DEF`). A table per expansion connects them, with the animation groups, the offsets and
  the fixes. The tables are generated once from the executable, then edited as source: a fix is a
  visible, reviewed diff.
- **Real-asset gate.** CI never sees an original file. A local gate decodes every entry of the
  user's archives, checks every name in the tables, and measures how long loading takes.
- **Record.** The server's record is one viewer's message log. The viewer plays records; a live
  game is the same messages arriving over a socket. An engine replay becomes a record by running
  the protocol's session in-process.
- **Reference frames.** The original, run from a copy, shows a situation; its frame is captured
  and compared with ours exactly. Differences become either fixes in our code or entries in the
  tables, each with its source.

## Milestones

Each milestone ends at a checkpoint with the user, after a fresh reviewer agent checked it. The
autonomy contract (lane HANDOFF) says what the agent decides alone in between.

### Closed milestones M0 to M3

Moved to `E17-archive.md` (verbatim; the full records are in `E17-log.md`, the design in `E17-design.md`):

| Milestone | Done | Content |
|---|---|---|
| M0 — the workspace | 2026-10-12 | `clients/` as its own workspace, the gates (`clients/scripts/check.sh`, `clients.yml`), `h3-view gate`, ADR-0020 |
| M1 — assets and a battle still | 2026-10-13 | code readings, the decoders (palettes, PCX, DEF, MSK), the real-asset gate, a battle still |
| M2 — the battle replay viewer | 2026-10-14 | the capture harness, the window/input/audio choice (ADR-0024), the scene model, the replay viewer |
| M3 — fidelity | 2026-10-23 (T9d) | the scene over time completed, recordings against the original under Wine, the string catalogue, the lane split (CLT-Q18) |

### M4 — sieges

Refined at T0 (2026-10-23) with the user; the tasks in `docs/lanes/client/QUEUE.md`. The engine
plays sieges since E04c (`Siege` with the wall parts' hit points, the gate and the moat;
`WallShot`, `WallQuaked`, `WallDestroyed`, `GateOpened`, `GateClosed`, `MoatDamage`; the turrets
as stacks at hexes 251, 254, 255; BAT-SIE-*); the viewer draws none of it yet. The original's
siege drawing rests on tables the battle lane read (`installation.md` §7.27 to §7.31): per town
18 wall parts (`siege_wall_parts_by_town` 0x66D868: hit points, screen point, the hex after which
each is drawn, six pictures), the turrets (0x63CF88), the backgrounds (0x63D2A0); the pictures
are 28 PCX per town (`Sg<town><part>.pcx`: the background, five walls in two states, the arch,
two towers and the keep with their caps, the top wall, the moat and its lip, four drawbridge
states; the Tower 26, without the moat).

- **T1a, the siege still read:** the redraw's siege branches (`battle_redraw` 0x494440's parts,
  the row passes' pass 0, the moat's second call, the towers last), which picture a part shows by
  its hit points, what the background buffer composes once; compared with VCMI; the town tables
  harvested into the asset tables (moved into T1b by the user at T1a). It says which towns
  follow the same rules (the user: then only a subset is captured in T1b).
- **T1b, the siege still captured:** the situation writer places a town (faction, fort level),
  the harness steps the attacker in, the viewer draws the background, walls, gate, moat, towers,
  turret shooters and the Catapult; situations for every town with a Castle, one Fort, one
  Citadel; **damaged and destroyed walls as stills** (the user, 2026-10-23): the harness sets
  the walls' hit points in the original's memory at the start and the situation gives the viewer
  the same values, on Castle and Conflux (T1a: the rules are the same in every town).
- **T2a, the siege over time read:** the Catapult's shot (`wall_shot_animate` 0x445FD0), a
  turret's (`turret_shot` 0x465AD0), the drawbridge (`gate_open` 0x4671C0, `gate_close`
  0x4672E0: the engine reports `GateOpened` after the step, the original opens the gate
  before it),
  Earthquake (0x5A7C80), the moat's damage, the Tower's mines, the siege's cursors, status lines
  and log lines.
- **T2b, the Catapult and the turrets recorded:** hot-seat siege sequences with the original's
  rolls forced (hits, misses, a wall and a tower destroyed; the keep's and the towers' shots).
- **T2c, the gate, the moat, Earthquake and the mouse recorded** (cut in two by the user,
  2026-10-24: T2c-1 the gate, the moat and a mine; T2c-2 Earthquake, the shots and the mouse): a
  defender out through the gate and back, a stack in the moat, a mine, Earthquake, a hero with
  Ballistics aiming the Catapult, a shot through the walls; the siege cursors.
- **T3, M4's closure:** a fresh reviewer, every capture afresh, the review list, M5 refined.
- **Exit:** the siege situations and sequences pass, or each difference is a listed fix or a
  classified quirk.
- **T0 done 2026-10-23** ([log](E17-log.md#m4-t0--m4-sieges-refined-2026-10-23)): the cut
  above agreed; the wall states as stills by memory, on a subset of towns if the rules agree.
- **T1a done 2026-10-23** ([log](E17-log.md#m4-t1a--the-siege-still-read-2026-10-23), design
  §10.1): the siege still read (the 18 parts and their picture states, the drawbridge and its
  rope, the walls drawn in three strips across the rows, the turrets' shooters, the moat over a
  stack), the same rules in every town; VCMI differs on a few points; the harvest moved into
  T1b, the wall states on Castle and Conflux (the user).
- **T1b done 2026-10-23** ([log](E17-log.md#m4-t1b--the-siege-still-captured-2026-10-23), design
  §10.1 "Captured"; not split, the user): the town tables harvested (`[[town]]`), the siege
  drawn (`h3_view::siege`), 15 siege situations exact: every town with a Castle, a Fort, a
  Citadel, and the walls hit and destroyed on Castle and Conflux (memory writes); hot seat with
  commanded war machines (the record opens after automatic shots), the Tower by the defender's
  record (its mines).
- **T2a done 2026-10-23** ([log](E17-log.md#m4-t2a--the-siege-over-time-read-2026-10-23), design
  §10.2; not split, the user): what moves in a siege read (the wall shot, the turret's shot, the
  drawbridge, the moat, the mines, Earthquake, the mouse's siege kinds, the end tune's town);
  T2b's and T2c's sequences listed; CLT-Q19 to Q21 answered (the slips fixed, the original's for
  the comparisons; the opening's automatic acts recorded by the server lane).
- **T2b done 2026-10-23** ([log](E17-log.md#m4-t2b--the-catapult-and-the-turrets-recorded-2026-10-23),
  design §10.2 "Recorded"; not split, the user): the turrets' and the wall shots' animation
  (the rock's arc, the explosions over the redraws, the parts' states at frame 5, CLT-Q19 (1) to
  (5) under `Bugs::Original`); seven sequences recorded (`keep`, `hit`, `missed`, `automatic`,
  `breach`, `tower`, `cyclops`), every timeline as recorded, the frames exact but the listed
  differences (the first frame's outline row, the explosions' shadows, the status line's hint,
  one stale square); CLT-Q22 asked (the explosion cut at the top edge, built as recommended).
- **T2c-1 done 2026-10-24** ([log](E17-log.md#m4-t2c-1--the-gate-the-moat-and-a-mine-recorded-2026-10-24),
  design §10.2 "Recorded (M4 T2c-1)"; T2c cut in two by the user): the drawbridge (the walk's
  stop and restart, the closing, its rectangle alone: CLT-Q23 asked, built as recommended), the
  moat's damage and re-entry, a mine's blast mid-walk; `gate`, `gate-fortress`, `moat` and
  `mine` as recorded, their frames exact but the listed differences; found: a reaction's hold
  without a striker, a standing wall's hex taking a count box's place.

### The adventure lane's harness (H1, H2: its batch E3)

Taken ahead of M3's remainder on 2026-10-19 (the user): the adventure lane's E08 T22a waits on
it (`docs/rules/surveys/E08-battles.md` §13; the requests in the queue's inbox since 2026-10-13).

- **H1 done 2026-10-19** ([log](E17-log.md#h1--questions-the-difficulty-battles-from-the-map-2026-10-19),
  [design §6.3](E17-design.md)): `h3adv.py` answers a visit's questions by the record their text
  was written from (`h3lod.py`, the topmost dialog's text in memory), starts a game at a chosen
  difficulty, plays the battles a step opens (auto combat, a retreat or a surrender by side) with
  their stacks read at the start and the end, and records heroes' details, the players' resources
  before and after and a tile's monster from memory; `setup.sh` turns on the auto combat of
  creatures. The check recipe (`capture/batches/harness-check.toml`) plays each; batch 2 still
  reproduces.
- **H2 done 2026-10-19** ([log](E17-log.md#h2--the-underground-more-heroes-the-starts-reads-a-second-day-2026-10-19)):
  a `walk` onto either level (the view switched by the panel's button; shown on a shipped map),
  the hero list scrolled beyond five heroes, a level-up's skill chosen, the heroes off the map's
  experience, the date; the check recipe plays a second day.

### Later (refined when they come up)

After M4 (sieges, above), **M5,
live play** (a battle played: hot seat in-process first, then against a server: the content
check, accounts; the server lane's notes in `NOTES.md`), in that order (the user, 2026-10-23,
T9c). Later: HD layouts, graphics options beyond `faithful`, the client's options (volumes),
video. The adventure map and the full game viewer: E24, the map-client lane.

## Research items from the asset notes

| Item | Where |
|---|---|
| 1. Archive precedence (SoD against AB), name lookup | M1 T1 (done: design §3.1) |
| 2. The off-colour pixels; the screen format | M1 T1 (done: design §3.2) |
| 3. Licences of MMArchive and homm3-unpacker before reading their code | stop the line: a question first, when a task wants them |
| 4. The video archives | later |
| 5. Capturing the original | M1 T8 (done: Wine in the sandbox, design §6); M2 T1 (done: the harness, §6.1) |
| 6. Per-expansion content and the ruleset profiles | M1 T4 |
| 7. The HD patch as a source of fixes and for HD | M3, later |
| 8. Shadows and the other keyed colours | M1 T1 (done: design §3.3) |
| 9. The adventure map's object draw order (the user: neither they nor VCMI got it exactly right) | the map-client lane (E24) since the split (2026-10-23); M1 T5 read only the battle's |

## Risks and known gaps

- **Graphics per expansion.** The Complete edition holds only SoD-era graphics: a RoE ruleset
  still shows SoD art.
- **Engine churn.** `h3-protocol` depends on `h3-battle`; the engine's frequent changes rebuild the
  client and may break it. The client CI job runs when those crates change (since T7 the client
  builds `h3-protocol`, `h3-pack-battle` and their engine dependencies).
- **The sandbox needs the user's computer online** for the original files (staged one LOD per
  call; the captures also need the five DLLs, the sound archives and `H3ab_ahd.vid`).
- **Wine is not Windows.** The captures run on Wine's DirectDraw; the exact match of the menu and
  the battlefield says it changes nothing there, but a difference only Wine could cause (timing,
  palette effects) would need a check on Windows.
- **A record does not say who acts** (T4b-1): the original ticks its idle steps and outlines the
  active stack differently on a player's turn and the computer's; the viewer guesses from the
  commands' times (design §7.9). A record of a live game may need the seats' kinds.
- **The background variant (CLT-Q4).** The original picks a battlefield's picture from the
  adventure map around the battle; a battle's record does not carry it, so the viewer shows the
  picture of an empty surrounding.
- **Checkpoint loading needs an engine change (CLT-Q2).** Until the battle lane's table-only
  reader lands, the client reads whole archives (212 MiB, about 0.2 s warm).

## Review lists

Accepted by the user (M1 2026-10-13, M2 2026-10-14, M3 2026-10-23); in the log,
[`E17-log.md`](E17-log.md#accepted-review-lists-m1-m2-m3) (M1 and M2 moved there on 2026-10-20,
M3 at its review, T9b). The next milestone's list starts here.

**M4** (for its closure, T3):

1. The asset tables' `[[town]]` (T1b, `clients/README.md`): the background, the turrets (the
   shooter by its original id, its drawer's `shift` harvested as one number, 44 for a two-hex
   creature less the Medusa's 5) and the 18 parts by index with five pictures each, an empty
   name for none.
2. The capture set's siege keys (T1b, the runbook): `key` (the step into the battle), `seat` (a
   side's record), `walls` (the wall states written into the original, the viewer's override
   `h3-view render --walls`; *provisional* until a record carries a hit part's state, T2b).
3. The siege situations are hot seat with both heroes commanding their war machines (T1b): the
   record opens after the machines' own acts (`BattleState::new` drops the start's events).
4. A record's hit points give a part's picture state only whole or destroyed (`Walls::of`): a
   hit (1) needs the timeline (T2b).
5. The asset tables' `siege_sounds` (T2c-1: the drawbridge's and a mine's blast's, harvested by
   name) and the key family `moat` (ARRAYTXT's nine moat names by town type, keyed by faction).
6. A siege's sounds in the window (T2c-1): the drawbridge's and the blast's play in category 2;
   T2b's `WallHit` and `WallMiss` (category 0) are cued but not played yet (*provisional*);
   Earthquake's spell sound and `WallHit` are cued on no stack (T2c-2).
7. The siege's mouse (T2c-2): the cursor's own tests know the acting stack by the asset tables'
   original ids (the Catapult 145; the shooters the walls never halve, 34, 35, 136, 137, 149) and
   the bows by the wire's `no_distance_penalty` (`pointer::legal`, the binary); a wire flag for
   the walls' exemption would replace the ids (a protocol change, the server and battle lanes).
   The wall-shot targets the cursor offers are the server's `ShootWall` commands.
8. The key family `wall` (WALLS.TXT's parts by town type, `wall.<faction>.<part>` for the eight
   targets' parts) and `battle.hint.turret` (GENRLTXT 157); the harness's forced values in order
   (`roll:<name>:<a>,<b>,...`, `xdd.c`'s queue of 8) and its roll `quake_target` (T2c-2).
9. Earthquake with the slip fixed (T2c-2): the damage before frame 5's redraw and shown everywhere
   from it, as a wall shot's (the original's: after it, the new pictures from frame 6).
